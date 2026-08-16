import React from "react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useKeyboardShortcuts } from "./use-keyboard-shortcuts";
import { createWrapper, createAppState } from "../../../services/test-utils";
import { Accidental, Articulation, PlayerType } from "../../../services/types";
import {
  AppContext,
  AudioContext,
  initialAudioState,
  initialPluginInventoryState,
  initialTransportState,
  PluginInventoryContext,
  TransportContext,
} from "../../../services/state/context";

// ---------------------------------------------------------------------------
// Mock the actions module
// ---------------------------------------------------------------------------
vi.mock("../../../services/actions", () => ({
  actions: {
    score: {
      flow: {
        deleteBar: vi.fn().mockResolvedValue(128),
      },
      clef: {
        clear: vi.fn().mockResolvedValue(undefined),
      },
      playingTechnique: {
        remove: vi.fn().mockResolvedValue(undefined),
      },
      keySignature: {
        clearAtTick: vi.fn().mockResolvedValue(undefined),
      },
      tempo: {
        clearAtTick: vi.fn().mockResolvedValue(undefined),
      },
      tuplet: {
        remove: vi.fn().mockResolvedValue(undefined),
      },
      timeSignature: {
        clearAtTick: vi.fn().mockResolvedValue(undefined),
        update: vi.fn().mockResolvedValue(undefined),
      },
      tone: {
        shiftPitchByStep: vi.fn().mockResolvedValue(undefined),
        shiftPitchByOctave: vi.fn().mockResolvedValue(undefined),
        remove: vi.fn().mockResolvedValue(undefined),
        setAccidental: vi.fn().mockResolvedValue(undefined),
        setArticulation: vi.fn().mockResolvedValue(undefined),
      },
    },
    ui: {
      setActiveFlowId: vi.fn().mockResolvedValue(undefined),
      sequencer: {
        selectBar: vi.fn().mockResolvedValue(undefined),
        clearSelection: vi.fn().mockResolvedValue(undefined),
      },
    },
    transport: {
      auditionTone: vi.fn().mockResolvedValue(undefined),
      play: vi.fn().mockResolvedValue(undefined),
      pause: vi.fn().mockResolvedValue(undefined),
      seek: vi.fn().mockResolvedValue(undefined),
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
const SECOND_TRACK_ID = 301;
const TONE_ID = 999;
const SECOND_TONE_ID = 1000;
const TUPLET_ID = 2000;

const buildStateWithBarSelection = () =>
  createAppState({
    ui: {
      flow_id: FLOW_ID,
      sequencer: {
        selection: { type: "Bar", flow_id: FLOW_ID, tick: 256 },
      },
    } as never,
  });

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

const buildStateWithLaterFlowToneSelection = () => {
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
            stave_ids: [STAVE_ID, 101],
            count: null,
            volume: 100,
            pan: 0,
            mute: false,
            solo: false,
          },
        },
      },
      tracks: {
        by_id: {
          [SECOND_TRACK_ID]: {
            id: SECOND_TRACK_ID,
            entries: {
              by_tick: { [96]: [SECOND_TONE_ID] },
              by_id: {
                [SECOND_TONE_ID]: {
                  id: SECOND_TONE_ID,
                  tick: 96,
                  type: "Tone" as const,
                  duration: 32,
                  pitch: { int: 67, accidental: Accidental.Natural },
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
          flow_id: FLOW_ID + 1,
          instrument_id: INSTRUMENT_ID,
          stave_id: 101,
          track_id: SECOND_TRACK_ID,
          tone_id: SECOND_TONE_ID,
        },
      },
    } as never,
  });
};

const buildStateWithTupletToneSelection = () => {
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
              by_tick: { 64: [TUPLET_ID] },
              by_id: {
                [TUPLET_ID]: {
                  id: TUPLET_ID,
                  tick: 64,
                  type: "Tuplet" as const,
                  note_count: 3,
                  in_space_of: 2,
                  base_note_value: 8,
                  groupings: [3],
                  entries: {
                    by_tick: { 0: [TONE_ID] },
                    by_id: {
                      [TONE_ID]: {
                        id: TONE_ID,
                        tick: 0,
                        type: "Tone" as const,
                        duration: 16,
                        pitch: { int: 60, accidental: Accidental.Natural },
                        velocity: { int: 100 },
                        articulations: [],
                      },
                    },
                  },
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
          tuplet_id: TUPLET_ID,
        },
      },
    } as never,
  });
};

const buildStateWithTupletSelection = () => {
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
              by_tick: { 64: [TUPLET_ID] },
              by_id: {
                [TUPLET_ID]: {
                  id: TUPLET_ID,
                  tick: 64,
                  type: "Tuplet" as const,
                  note_count: 3,
                  in_space_of: 2,
                  base_note_value: 8,
                  groupings: [3],
                  entries: {
                    by_tick: { 0: [TONE_ID] },
                    by_id: {
                      [TONE_ID]: {
                        id: TONE_ID,
                        tick: 0,
                        type: "Tone" as const,
                        duration: 16,
                        pitch: { int: 60, accidental: Accidental.Natural },
                        velocity: { int: 100 },
                        articulations: [],
                      },
                    },
                  },
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
          type: "Tuplet",
          flow_id: FLOW_ID,
          instrument_id: INSTRUMENT_ID,
          stave_id: STAVE_ID,
          track_id: TRACK_ID,
          tuplet_id: TUPLET_ID,
          tick: 64,
        },
      },
    } as never,
  });
};

const buildStateWithTupletRestSelection = () => {
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
              by_tick: { 64: [TUPLET_ID] },
              by_id: {
                [TUPLET_ID]: {
                  id: TUPLET_ID,
                  tick: 64,
                  type: "Tuplet" as const,
                  note_count: 3,
                  in_space_of: 2,
                  base_note_value: 8,
                  groupings: [3],
                  entries: {
                    by_tick: {},
                    by_id: {},
                  },
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
          type: "Rest",
          flow_id: FLOW_ID,
          instrument_id: INSTRUMENT_ID,
          stave_id: STAVE_ID,
          track_id: TRACK_ID,
          tick: 0,
          tuplet_id: TUPLET_ID,
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

const dispatchKeyFromElement = (
  element: HTMLElement,
  key: string,
  options?: KeyboardEventInit,
) => {
  act(() => {
    element.dispatchEvent(
      new KeyboardEvent("keydown", { key, bubbles: true, ...options }),
    );
  });
};

describe("useKeyboardShortcuts", () => {
  let consoleError: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.clearAllMocks();
    consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
  });

  afterEach(() => {
    vi.clearAllMocks();
    consoleError.mockRestore();
  });

  it("mounts without errors when no selection", () => {
    expect(() => {
      renderHook(() => useKeyboardShortcuts(), {
        wrapper: createWrapper({ appState: createAppState() }),
      });
    }).not.toThrow();
  });

  it("toggles pause on Space when transport is already playing", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: buildStateWithToneSelection(),
        transportState: { playing: true, current_tick: 64 },
      }),
    });

    await act(async () => {
      dispatchKey(" ");
    });

    expect(actions.transport.pause).toHaveBeenCalledOnce();
    expect(actions.transport.play).not.toHaveBeenCalled();
  });

  it("toggles play on Space when transport is stopped", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: buildStateWithToneSelection(),
        transportState: { playing: false, current_tick: 64 },
      }),
    });

    await act(async () => {
      dispatchKey(" ");
    });

    expect(actions.transport.play).toHaveBeenCalledOnce();
    expect(actions.transport.pause).not.toHaveBeenCalled();
  });

  it("seeks to a selected rest tick and plays on P", async () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Rest",
            flow_id: FLOW_ID,
            instrument_id: INSTRUMENT_ID,
            stave_id: STAVE_ID,
            track_id: TRACK_ID,
            tick: 128,
          },
        },
      } as never,
    });

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: state,
        transportState: { playing: false, current_tick: 64 },
      }),
    });

    await act(async () => {
      dispatchKey("p");
    });

    expect(actions.transport.seek).toHaveBeenCalledWith(128);
    expect(actions.transport.play).toHaveBeenCalledOnce();
  });

  it("switches to the selected flow before playing from selection on P", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: buildStateWithLaterFlowToneSelection(),
        transportState: { playing: false, current_tick: 0 },
      }),
    });

    await act(async () => {
      dispatchKey("p");
    });

    expect(actions.ui.setActiveFlowId).toHaveBeenCalledWith(FLOW_ID + 1);
    expect(actions.transport.seek).toHaveBeenCalledWith(96);
    expect(actions.transport.play).toHaveBeenCalledOnce();
  });

  it("switches to the selected flow before playing from a selected rest on P", async () => {
    const state = createAppState({
      ui: {
        flow_id: FLOW_ID,
        sequencer: {
          selection: {
            type: "Rest",
            flow_id: FLOW_ID + 1,
            instrument_id: INSTRUMENT_ID,
            stave_id: STAVE_ID,
            track_id: TRACK_ID,
            tick: 224,
          },
        },
      },
    } as never);

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: state,
        transportState: { playing: false, current_tick: 0 },
      }),
    });

    await act(async () => {
      dispatchKey("p");
    });

    expect(actions.ui.setActiveFlowId).toHaveBeenCalledWith(FLOW_ID + 1);
    expect(actions.transport.seek).toHaveBeenCalledWith(224);
    expect(actions.transport.play).toHaveBeenCalledOnce();
  });

  it("seeks to a selected host tone tick and plays on P", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: buildStateWithToneSelection(),
        transportState: { playing: false, current_tick: 0 },
      }),
    });

    await act(async () => {
      dispatchKey("p");
    });

    expect(actions.transport.seek).toHaveBeenCalledWith(64);
    expect(actions.transport.play).toHaveBeenCalledOnce();
  });

  it("seeks to the host tuplet tick plus local tone offset and plays on P", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: buildStateWithTupletToneSelection(),
        transportState: { playing: false, current_tick: 0 },
      }),
    });

    await act(async () => {
      dispatchKey("p");
    });

    expect(actions.transport.seek).toHaveBeenCalledWith(64);
    expect(actions.transport.play).toHaveBeenCalledOnce();
  });

  it("seeks to the tuplet host tick when a selected tuplet rest starts at local tick zero", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: buildStateWithTupletRestSelection(),
        transportState: { playing: false, current_tick: 0 },
      }),
    });

    await act(async () => {
      dispatchKey("p");
    });

    expect(actions.transport.seek).toHaveBeenCalledWith(64);
    expect(actions.transport.play).toHaveBeenCalledOnce();
  });

  it("falls back to pause on P when there is no selection and transport is playing", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: createAppState(),
        transportState: { playing: true, current_tick: 64 },
      }),
    });

    await act(async () => {
      dispatchKey("p");
    });

    expect(actions.transport.pause).toHaveBeenCalledOnce();
    expect(actions.transport.seek).not.toHaveBeenCalled();
  });

  it("falls back to play on P when there is no selection and transport is stopped", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: createAppState(),
        transportState: { playing: false, current_tick: 64 },
      }),
    });

    await act(async () => {
      dispatchKey("p");
    });

    expect(actions.transport.play).toHaveBeenCalledOnce();
    expect(actions.transport.seek).not.toHaveBeenCalled();
  });

  it("does not dispatch tone shortcuts when a tempo selection is active", async () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tempo",
            flow_id: FLOW_ID,
            tempo_id: 555,
            tick: 64,
          },
        },
      } as never,
    });

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
    });

    expect(actions.score.tone.shiftPitchByStep).not.toHaveBeenCalled();
  });

  it("returns null selectionInfo when Tone selection references missing track", () => {
    // Selection points to track 9999 which does not exist → line 27 branch
    const state = createAppState({
      score: {
        tracks: { by_id: {} }, // empty — track 9999 missing
      } as never,
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: 5,
            instrument_id: 10,
            stave_id: 100,
            track_id: 9999,
            tone_id: 999,
          },
        },
      } as never,
    });

    // The hook should mount without errors even with missing track
    expect(() => {
      renderHook(() => useKeyboardShortcuts(), {
        wrapper: createWrapper({ appState: state }),
      });
    }).not.toThrow();
  });

  it("returns null selectionInfo when Tone entry is missing from track", () => {
    // Track exists but entry 9999 does not → line 32 branch
    const state = createAppState({
      score: {
        tracks: {
          by_id: {
            300: {
              id: 300,
              entries: { by_tick: {}, by_id: {} }, // tone_id 999 missing
            },
          },
        },
      } as never,
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: 5,
            instrument_id: 10,
            stave_id: 100,
            track_id: 300,
            tone_id: 999, // missing from by_id
          },
        },
      } as never,
    });

    expect(() => {
      renderHook(() => useKeyboardShortcuts(), {
        wrapper: createWrapper({ appState: state }),
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
      TRACK_ID,
      null,
      TONE_ID,
      1,
    );
    expect(actions.transport.auditionTone).not.toHaveBeenCalled();
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
      TRACK_ID,
      null,
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
      null,
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

    expect(actions.score.tone.remove).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
    );
  });

  it("continues to target a surviving later-flow tone selection", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: buildStateWithLaterFlowToneSelection(),
      }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
    });

    expect(actions.score.tone.shiftPitchByStep).toHaveBeenCalledWith(
      SECOND_TRACK_ID,
      null,
      SECOND_TONE_ID,
      1,
    );
  });

  it("dispatches pitch shortcuts for a selected tuplet-owned tone", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: buildStateWithTupletToneSelection(),
      }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
    });

    expect(actions.score.tone.shiftPitchByStep).toHaveBeenCalledWith(
      TRACK_ID,
      TUPLET_ID,
      TONE_ID,
      1,
    );
  });

  it("does not dispatch edit or delete actions once selection has been cleared", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: createAppState({
          ui: {
            sequencer: {
              selection: null,
            },
          } as never,
        }),
      }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Backspace" }));
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
    });

    expect(actions.score.tone.remove).not.toHaveBeenCalled();
    expect(actions.score.tone.shiftPitchByStep).not.toHaveBeenCalled();
  });

  it("ignores delete shortcuts from text inputs", async () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tempo",
            flow_id: FLOW_ID,
            tempo_id: 555,
            tick: 64,
          },
        },
      } as never,
    });

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    const input = document.createElement("input");
    input.type = "text";
    document.body.appendChild(input);

    try {
      await act(async () => {
        dispatchKeyFromElement(input, "Backspace");
      });
    } finally {
      input.remove();
    }

    expect(actions.score.tempo.clearAtTick).not.toHaveBeenCalled();
    expect(actions.ui.sequencer.clearSelection).not.toHaveBeenCalled();
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
      null,
      TONE_ID,
      Articulation.Staccato,
    );
  });

  it("dispatches shiftPitchByOctave(-1) on ArrowDown+Cmd+Option", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", {
          key: "ArrowDown",
          metaKey: true,
          altKey: true,
        }),
      );
    });

    expect(actions.score.tone.shiftPitchByOctave).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      -1,
    );
  });

  it("dispatches setAccidental(Sharp) on '='", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "=" }));
    });
    expect(actions.score.tone.setAccidental).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      Accidental.Sharp,
    );
  });

  it("dispatches setAccidental(Flat) on '-'", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "-" }));
    });
    expect(actions.score.tone.setAccidental).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      Accidental.Flat,
    );
  });

  it("dispatches setAccidental(Natural) on '0'", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "0" }));
    });
    expect(actions.score.tone.setAccidental).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      Accidental.Natural,
    );
  });

  it("dispatches remove on Delete", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));
    });
    expect(actions.score.tone.remove).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
    );
  });

  it("deletes a selected bar and selects the returned neighbor", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithBarSelection() }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));
    });

    expect(actions.score.flow.deleteBar).toHaveBeenCalledWith(FLOW_ID, 256);
    expect(actions.ui.sequencer.selectBar).toHaveBeenCalledWith(FLOW_ID, 128);
  });

  it("deletes selected tempo and clears selection", async () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tempo",
            flow_id: FLOW_ID,
            tempo_id: 555,
            tick: 64,
          },
        },
      } as never,
    });

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));
    });

    expect(actions.score.tempo.clearAtTick).toHaveBeenCalledWith(FLOW_ID, 64);
    expect(actions.ui.sequencer.clearSelection).not.toHaveBeenCalled();
  });

  it("clears selection when deleting the opening tempo", async () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tempo",
            flow_id: FLOW_ID,
            tempo_id: 555,
            tick: 0,
          },
        },
      } as never,
    });

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));
    });

    expect(actions.score.tempo.clearAtTick).toHaveBeenCalledWith(FLOW_ID, 0);
    expect(actions.ui.sequencer.clearSelection).not.toHaveBeenCalled();
  });

  it("deletes selected time signature and clears selection away from tick 0", async () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "TimeSignature",
            flow_id: FLOW_ID,
            time_signature_id: 777,
            tick: 64,
          },
        },
      } as never,
    });

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Backspace" }));
    });

    expect(actions.score.timeSignature.clearAtTick).toHaveBeenCalledWith(
      FLOW_ID,
      64,
    );
    expect(actions.ui.sequencer.clearSelection).not.toHaveBeenCalled();
  });

  it("leaves opening time signature delete cleanup to the backend", async () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "TimeSignature",
            flow_id: FLOW_ID,
            time_signature_id: 777,
            tick: 0,
          },
        },
      } as never,
    });

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));
    });

    expect(actions.score.timeSignature.clearAtTick).toHaveBeenCalledWith(
      FLOW_ID,
      0,
    );
    expect(actions.ui.sequencer.clearSelection).not.toHaveBeenCalled();
  });

  it("deletes selected key signature and clears selection", async () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "KeySignature",
            flow_id: FLOW_ID,
            key_signature_id: 778,
            tick: 64,
          },
        },
      } as never,
    });

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));
    });

    expect(actions.score.keySignature.clearAtTick).toHaveBeenCalledWith(
      FLOW_ID,
      64,
    );
    expect(actions.ui.sequencer.clearSelection).not.toHaveBeenCalled();
  });

  it("clears selection when deleting the opening key signature", async () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "KeySignature",
            flow_id: FLOW_ID,
            key_signature_id: 778,
            tick: 0,
          },
        },
      } as never,
    });

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));
    });

    expect(actions.score.keySignature.clearAtTick).toHaveBeenCalledWith(
      FLOW_ID,
      0,
    );
    expect(actions.ui.sequencer.clearSelection).not.toHaveBeenCalled();
  });

  it("deletes selected clef and clears selection away from tick 0", async () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Clef",
            flow_id: FLOW_ID,
            stave_id: STAVE_ID,
            clef_id: 779,
            tick: 64,
          },
        },
      } as never,
    });

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));
    });

    expect(actions.score.clef.clear).toHaveBeenCalledWith(
      FLOW_ID,
      64,
      STAVE_ID,
    );
    expect(actions.ui.sequencer.clearSelection).not.toHaveBeenCalled();
  });

  it("deletes the selected playing-technique entry by id", async () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "PlayingTechnique",
            flow_id: FLOW_ID,
            stave_id: STAVE_ID,
            playing_technique_id: 880,
            tick: 64,
          },
        },
      } as never,
    });

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));
    });

    expect(actions.score.playingTechnique.remove).toHaveBeenCalledWith(
      FLOW_ID,
      STAVE_ID,
      880,
    );
    expect(actions.ui.sequencer.clearSelection).not.toHaveBeenCalled();
  });

  it("does not dispatch tone-only edit shortcuts for a selected clef", async () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Clef",
            flow_id: FLOW_ID,
            stave_id: STAVE_ID,
            clef_id: 779,
            tick: 64,
          },
        },
      } as never,
    });

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
    });

    expect(actions.score.tone.shiftPitchByStep).not.toHaveBeenCalled();
  });

  it("dispatches setArticulation(Tenuto) on '\\'", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "\\" }));
    });
    expect(actions.score.tone.setArticulation).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      Articulation.Tenuto,
    );
  });

  it("dispatches setArticulation(Staccatissimo) on '['", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "[" }));
    });
    expect(actions.score.tone.setArticulation).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      Articulation.Staccatissimo,
    );
  });

  it('dispatches setArticulation(StaccatoTenuto) on "\'"', async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "'" }));
    });
    expect(actions.score.tone.setArticulation).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      Articulation.StaccatoTenuto,
    );
  });

  it("dispatches setArticulation(Accent) on '}'", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "}" }));
    });
    expect(actions.score.tone.setArticulation).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      Articulation.Accent,
    );
  });

  it("dispatches setArticulation(Marcato) on '|'", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "|" }));
    });
    expect(actions.score.tone.setArticulation).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      Articulation.Marcato,
    );
  });

  it("dispatches setArticulation(Stressed) on '{'", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "{" }));
    });
    expect(actions.score.tone.setArticulation).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      Articulation.Stressed,
    );
  });

  it("dispatches setArticulation(Unstressed) on '\"' ", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: '"' }));
    });
    expect(actions.score.tone.setArticulation).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      Articulation.Unstressed,
    );
  });

  it("dispatches remove for a selected tuplet-owned tone", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: buildStateWithTupletToneSelection(),
      }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));
    });

    expect(actions.score.tone.remove).toHaveBeenCalledWith(
      TRACK_ID,
      TUPLET_ID,
      TONE_ID,
    );
  });

  it("dispatches tuplet remove for a selected tuplet container", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: buildStateWithTupletSelection(),
      }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));
    });

    expect(actions.score.tuplet.remove).toHaveBeenCalledWith(
      TRACK_ID,
      TUPLET_ID,
    );
  });

  it("dispatches accidental shortcuts for a selected tuplet-owned tone", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: buildStateWithTupletToneSelection(),
      }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "=" }));
    });

    expect(actions.score.tone.setAccidental).toHaveBeenCalledWith(
      TRACK_ID,
      TUPLET_ID,
      TONE_ID,
      Accidental.Sharp,
    );
  });

  it("dispatches articulation shortcuts for a selected tuplet-owned tone", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({
        appState: buildStateWithTupletToneSelection(),
      }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "]" }));
    });

    expect(actions.score.tone.setArticulation).toHaveBeenCalledWith(
      TRACK_ID,
      TUPLET_ID,
      TONE_ID,
      Articulation.Staccato,
    );
  });

  it("shifts the selected tone upward by step when the shortcut is pressed", async () => {
    const PLUGIN_ID = "plugin-abc";
    const state = buildStateWithToneSelection();
    (state.score.instruments.by_id as Record<number, unknown>)[INSTRUMENT_ID] =
      {
        id: INSTRUMENT_ID,
        definition_id: "piano",
        instrument_type: 0,
        long_name: "Piano",
        short_name: "Pno.",
        stave_ids: [STAVE_ID],
        count: null,
        volume: 100,
        pan: 0,
        mute: false,
        solo: false,
      };
    state.resolved_playback_route_by_instrument_id = {
      [INSTRUMENT_ID]: {
        status: 1,
        slot_id: 77,
        slot_target: { plugin_id: PLUGIN_ID },
        effective_plugin_id: PLUGIN_ID,
        target: null,
        address: null,
      },
    };
    state.single_instrument_transport_ready = true;
    state.preferences = { ...state.preferences, audition: true };

    vi.mocked(actions.score.tone.shiftPitchByStep).mockResolvedValueOnce(61);

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
    });

    expect(actions.score.tone.shiftPitchByStep).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      1,
    );
    expect(actions.transport.auditionTone).toHaveBeenCalledWith(
      INSTRUMENT_ID,
      61,
    );
  });

  it("still shifts pitch when audition preference is disabled", async () => {
    const PLUGIN_ID = "plugin-abc";
    const state = buildStateWithToneSelection();
    (state.score.instruments.by_id as Record<number, unknown>)[INSTRUMENT_ID] =
      {
        id: INSTRUMENT_ID,
        definition_id: "piano",
        instrument_type: 0,
        long_name: "Piano",
        short_name: "Pno.",
        stave_ids: [STAVE_ID],
        count: null,
        volume: 100,
        pan: 0,
        mute: false,
        solo: false,
      };
    state.resolved_playback_route_by_instrument_id = {
      [INSTRUMENT_ID]: {
        status: 1,
        slot_id: 77,
        slot_target: { plugin_id: PLUGIN_ID },
        effective_plugin_id: PLUGIN_ID,
        target: null,
        address: null,
      },
    };
    state.single_instrument_transport_ready = true;
    state.preferences = { ...state.preferences, audition: false };

    vi.mocked(actions.score.tone.shiftPitchByStep).mockResolvedValueOnce(61);

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
    });

    expect(actions.score.tone.shiftPitchByStep).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      1,
    );
    expect(actions.transport.auditionTone).not.toHaveBeenCalled();
  });

  it("auditions a newly selected tone when selection changes to a ready routed instrument", async () => {
    const initialState = createAppState({
      ui: {
        sequencer: {
          selection: null,
        },
      } as never,
    });
    const selectedState = buildStateWithToneSelection();
    selectedState.resolved_playback_route_by_instrument_id = {
      [INSTRUMENT_ID]: {
        status: 1,
        slot_id: 77,
        slot_target: { plugin_id: "plugin-abc" },
        effective_plugin_id: "plugin-abc",
        target: null,
        address: null,
      },
    };
    selectedState.single_instrument_transport_ready = true;

    let currentAppState = initialState;

    const Wrapper = ({ children }: { children: React.ReactNode }) => {
      return React.createElement(
        AppContext.Provider,
        { value: currentAppState },
        React.createElement(
          AudioContext.Provider,
          { value: structuredClone(initialAudioState) },
          React.createElement(
            PluginInventoryContext.Provider,
            { value: structuredClone(initialPluginInventoryState) },
            React.createElement(
              TransportContext.Provider,
              { value: structuredClone(initialTransportState) },
              children,
            ),
          ),
        ),
      );
    };

    const { rerender } = renderHook(() => useKeyboardShortcuts(), {
      wrapper: Wrapper,
    });

    currentAppState = selectedState;
    rerender();

    await act(async () => {
      await Promise.resolve();
    });

    expect(actions.transport.auditionTone).toHaveBeenCalledWith(
      INSTRUMENT_ID,
      60,
    );
  });

  it("still shifts pitch when instrument has no assigned plugin", async () => {
    const state = buildStateWithToneSelection();
    state.preferences = { ...state.preferences, audition: true };

    vi.mocked(actions.score.tone.shiftPitchByStep).mockResolvedValueOnce(62);

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
    });

    expect(actions.score.tone.shiftPitchByStep).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      1,
    );
  });

  it("auditions the selected tone when its route becomes ready even if transport trust stays false", async () => {
    const initialState = buildStateWithToneSelection();
    initialState.preferences = { ...initialState.preferences, audition: true };
    initialState.single_instrument_transport_ready = false;

    const readyState = structuredClone(initialState);
    readyState.resolved_playback_route_by_instrument_id = {
      [INSTRUMENT_ID]: {
        status: 1,
        slot_id: 77,
        slot_target: { plugin_id: "plugin-abc" },
        effective_plugin_id: "plugin-abc",
        target: null,
        address: null,
      },
    };

    let currentAppState = initialState;

    const Wrapper = ({ children }: { children: React.ReactNode }) => {
      return React.createElement(
        AppContext.Provider,
        { value: currentAppState },
        React.createElement(
          AudioContext.Provider,
          { value: structuredClone(initialAudioState) },
          React.createElement(
            PluginInventoryContext.Provider,
            { value: structuredClone(initialPluginInventoryState) },
            React.createElement(
              TransportContext.Provider,
              { value: structuredClone(initialTransportState) },
              children,
            ),
          ),
        ),
      );
    };

    const { rerender } = renderHook(() => useKeyboardShortcuts(), {
      wrapper: Wrapper,
    });

    currentAppState = readyState;
    rerender();

    await act(async () => {
      await Promise.resolve();
    });

    expect(actions.transport.auditionTone).toHaveBeenCalledWith(
      INSTRUMENT_ID,
      60,
    );
  });

  it("auditions pitch edits when the route is ready even if single-instrument transport trust is false", async () => {
    const state = buildStateWithToneSelection();
    state.single_instrument_transport_ready = false;
    state.preferences = { ...state.preferences, audition: true };
    state.resolved_playback_route_by_instrument_id = {
      [INSTRUMENT_ID]: {
        status: 1,
        slot_id: 77,
        slot_target: { plugin_id: "plugin-abc" },
        effective_plugin_id: "plugin-abc",
        target: null,
        address: null,
      },
    };

    vi.mocked(actions.score.tone.shiftPitchByStep).mockResolvedValueOnce(61);

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
      await Promise.resolve();
    });

    expect(actions.score.tone.shiftPitchByStep).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      1,
    );
    expect(actions.transport.auditionTone).toHaveBeenCalledWith(
      INSTRUMENT_ID,
      61,
    );
  });

  it("logs audition failures without blocking the pitch edit", async () => {
    const state = buildStateWithToneSelection();
    state.resolved_playback_route_by_instrument_id = {
      [INSTRUMENT_ID]: {
        status: 1,
        slot_id: 77,
        slot_target: { plugin_id: "plugin-abc" },
        effective_plugin_id: "plugin-abc",
        target: null,
        address: null,
      },
    };
    state.single_instrument_transport_ready = true;
    state.preferences = { ...state.preferences, audition: true };

    const error = new Error("audition failed");
    vi.mocked(actions.score.tone.shiftPitchByStep).mockResolvedValueOnce(61);
    vi.mocked(actions.transport.auditionTone).mockRejectedValueOnce(error);

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
      await Promise.resolve();
    });

    expect(actions.score.tone.shiftPitchByStep).toHaveBeenCalledWith(
      TRACK_ID,
      null,
      TONE_ID,
      1,
    );
    expect(consoleError).toHaveBeenCalledWith("Failed to audition tone", error);
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
