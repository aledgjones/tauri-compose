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
      clef: {
        clear: vi.fn().mockResolvedValue(undefined),
      },
      keySignature: {
        clearAtTick: vi.fn().mockResolvedValue(undefined),
      },
      tempo: {
        clearAtTick: vi.fn().mockResolvedValue(undefined),
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
const SECOND_TRACK_ID = 301;
const TONE_ID = 999;
const SECOND_TONE_ID = 1000;

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
            assigned_plugin_id: null,
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
      SECOND_TONE_ID,
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
    expect(actions.score.tone.remove).toHaveBeenCalledWith(TRACK_ID, TONE_ID);
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
      TONE_ID,
      Articulation.Unstressed,
    );
  });

  it("calls audition when shouldAudition=true and result is a number with assigned plugin", async () => {
    const PLUGIN_ID = "plugin-abc";
    const state = buildStateWithToneSelection();
    // give the instrument an assigned plugin so audition fires
    (state.score.instruments.by_id as Record<number, unknown>)[INSTRUMENT_ID] =
      {
        id: INSTRUMENT_ID,
        definition_id: "piano",
        instrument_type: 0,
        long_name: "Piano",
        short_name: "Pno.",
        stave_ids: [STAVE_ID],
        count: null,
        assigned_plugin_id: PLUGIN_ID,
        volume: 100,
        pan: 0,
        mute: false,
        solo: false,
      };
    state.preferences = { ...state.preferences, audition: true };

    // shiftPitchByStep returns a new pitch int (number) → audition should fire
    vi.mocked(actions.score.tone.shiftPitchByStep).mockResolvedValueOnce(61);

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
    });

    expect(actions.sampler.audition).toHaveBeenCalledWith(PLUGIN_ID, 61);
  });

  it("does not audition when audition preference is disabled", async () => {
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
        assigned_plugin_id: PLUGIN_ID,
        volume: 100,
        pan: 0,
        mute: false,
        solo: false,
      };
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

    expect(actions.sampler.audition).not.toHaveBeenCalled();
  });

  it("does not audition when instrument has no assigned plugin", async () => {
    const state = buildStateWithToneSelection();
    state.preferences = { ...state.preferences, audition: true };
    // instrument has no assigned plugin (null by default in buildStateWithToneSelection)

    vi.mocked(actions.score.tone.shiftPitchByStep).mockResolvedValueOnce(62);

    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: state }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
    });

    expect(actions.sampler.audition).not.toHaveBeenCalled();
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
