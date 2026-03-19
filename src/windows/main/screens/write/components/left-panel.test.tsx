import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import {
  renderWithProviders,
  createAppState,
} from "../../../../../services/test-utils";
import { LeftPanel } from "./left-panel";
import { Accidental, Articulation } from "../../../../../services/types";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    score: {
      tone: {
        setDuration: vi.fn(),
        setAccidental: vi.fn(),
        removeAccidental: vi.fn(),
        setArticulation: vi.fn(),
        removeArticulation: vi.fn(),
      },
    },
  },
}));

import { actions } from "../../../../../services/actions";

describe("LeftPanel", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders all duration buttons", () => {
    renderWithProviders(<LeftPanel spanRow={false} />);
    expect(screen.getByTitle("Whole")).toBeInTheDocument();
    expect(screen.getByTitle("Half")).toBeInTheDocument();
    expect(screen.getByTitle("Quarter")).toBeInTheDocument();
    expect(screen.getByTitle("Eighth")).toBeInTheDocument();
    expect(screen.getByTitle("Sixteenth")).toBeInTheDocument();
    expect(screen.getByTitle("Thirty-Second")).toBeInTheDocument();
  });

  it("renders accidental buttons", () => {
    renderWithProviders(<LeftPanel spanRow={false} />);
    expect(screen.getByTitle("Natural")).toBeInTheDocument();
    expect(screen.getByTitle("Flat")).toBeInTheDocument();
    expect(screen.getByTitle("Sharp")).toBeInTheDocument();
  });

  it("renders articulation buttons", () => {
    renderWithProviders(<LeftPanel spanRow={false} />);
    expect(screen.getByTitle("Staccato")).toBeInTheDocument();
    expect(screen.getByTitle("Tenuto")).toBeInTheDocument();
    expect(screen.getByTitle("Accent")).toBeInTheDocument();
    expect(screen.getByTitle("Marcato")).toBeInTheDocument();
  });

  it("duration buttons are disabled when no tone selected", () => {
    renderWithProviders(<LeftPanel spanRow={false} />);
    const wholeBtn = screen.getByTitle("Whole");
    expect(wholeBtn.className).toContain("pointer-events-none");
  });

  it("applies row-span-2 when spanRow is true", () => {
    const { container } = renderWithProviders(<LeftPanel spanRow={true} />);
    expect(container.firstChild).toHaveClass("row-span-2");
  });

  it("does not apply row-span-2 when spanRow is false", () => {
    const { container } = renderWithProviders(<LeftPanel spanRow={false} />);
    expect(container.firstChild).not.toHaveClass("row-span-2");
  });

  it("duration buttons are enabled when a tone is selected", () => {
    const trackId = 1;
    const toneId = 2;
    const appState = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: 0,
            instrument_id: 0,
            stave_id: 0,
            track_id: trackId,
            tone_id: toneId,
          },
        },
      } as never,
      score: {
        tracks: {
          by_id: {
            [trackId]: {
              id: trackId,
              entries: {
                by_tick: {},
                by_id: {
                  [toneId]: {
                    id: toneId,
                    type: "Tone",
                    tick: 0,
                    duration: 768,
                    velocity: { int: 100 },
                    pitch: { int: 60, accidental: Accidental.Natural },
                    articulations: [],
                  },
                },
              },
            },
          },
        },
      } as never,
    });
    renderWithProviders(<LeftPanel spanRow={false} />, { appState });
    const quarterBtn = screen.getByTitle("Quarter");
    expect(quarterBtn.className).not.toContain("pointer-events-none");
  });

  it("duration buttons stay enabled for ordinary tones when selection carries tuplet_id as null", () => {
    const trackId = 1;
    const toneId = 2;
    const appState = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: 0,
            instrument_id: 0,
            stave_id: 0,
            track_id: trackId,
            tone_id: toneId,
            tuplet_id: null,
          },
        },
      } as never,
      score: {
        tracks: {
          by_id: {
            [trackId]: {
              id: trackId,
              entries: {
                by_tick: {},
                by_id: {
                  [toneId]: {
                    id: toneId,
                    type: "Tone",
                    tick: 0,
                    duration: 768,
                    velocity: { int: 100 },
                    pitch: { int: 60, accidental: Accidental.Natural },
                    articulations: [],
                  },
                },
              },
            },
          },
        },
      } as never,
    });

    renderWithProviders(<LeftPanel spanRow={false} />, { appState });
    const quarterBtn = screen.getByTitle("Quarter");
    expect(quarterBtn.className).not.toContain("pointer-events-none");
  });

  const makeStateWithTone = (
    durationOverride?: number,
    accidentalOverride?: Accidental,
    articulations: Articulation[] = [],
  ) => {
    const trackId = 1;
    const toneId = 2;
    return createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: 0,
            instrument_id: 0,
            stave_id: 0,
            track_id: trackId,
            tone_id: toneId,
          },
        },
      } as never,
      score: {
        tracks: {
          by_id: {
            [trackId]: {
              id: trackId,
              entries: {
                by_tick: {},
                by_id: {
                  [toneId]: {
                    id: toneId,
                    type: "Tone",
                    tick: 0,
                    duration: durationOverride ?? 16, // eighth note = 16 ticks
                    velocity: { int: 100 },
                    pitch: {
                      int: 60,
                      accidental: accidentalOverride ?? Accidental.Natural,
                    },
                    articulations,
                  },
                },
              },
            },
          },
        },
      } as never,
    });
  };

  const makeStateWithTupletTone = (
    durationOverride = 16,
    accidentalOverride: Accidental = Accidental.Natural,
    articulations: Articulation[] = [],
  ) => {
    const trackId = 1;
    const toneId = 2;
    const tupletId = 20;

    return createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: 0,
            instrument_id: 0,
            stave_id: 0,
            track_id: trackId,
            tone_id: toneId,
            tuplet_id: tupletId,
          },
        },
      } as never,
      score: {
        tracks: {
          by_id: {
            [trackId]: {
              id: trackId,
              entries: {
                by_tick: { 0: [tupletId] },
                by_id: {
                  [tupletId]: {
                    id: tupletId,
                    tick: 0,
                    type: "Tuplet",
                    note_count: 3,
                    in_space_of: 2,
                    base_note_value: 8,
                    groupings: [3],
                    entries: {
                      by_tick: { 0: [toneId] },
                      by_id: {
                        [toneId]: {
                          id: toneId,
                          type: "Tone",
                          tick: 0,
                          duration: durationOverride,
                          velocity: { int: 100 },
                          pitch: {
                            int: 60,
                            accidental: accidentalOverride,
                          },
                          articulations,
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
    });
  };

  it("calls setDuration when a duration button is clicked with tone selected", () => {
    renderWithProviders(<LeftPanel spanRow={false} />, {
      appState: makeStateWithTone(),
    });
    fireEvent.click(screen.getByTitle("Quarter"));
    expect(vi.mocked(actions.score.tone.setDuration)).toHaveBeenCalled();
  });

  it("calls setAccidental when an accidental button is clicked with tone selected", () => {
    renderWithProviders(<LeftPanel spanRow={false} />, {
      appState: makeStateWithTone(),
    });
    fireEvent.click(screen.getByTitle("Sharp"));
    expect(vi.mocked(actions.score.tone.setAccidental)).toHaveBeenCalled();
  });

  it("calls setArticulation when an articulation button is clicked with tone selected", () => {
    renderWithProviders(<LeftPanel spanRow={false} />, {
      appState: makeStateWithTone(),
    });
    fireEvent.click(screen.getByTitle("Staccato"));
    expect(vi.mocked(actions.score.tone.setArticulation)).toHaveBeenCalled();
  });

  it("duration button is selected when tone duration matches", () => {
    // Eighth note = NoteDuration.Eighth → ticks = 16
    renderWithProviders(<LeftPanel spanRow={false} />, {
      appState: makeStateWithTone(16),
    });
    const eighthBtn = screen.getByTitle("Eighth");
    expect(eighthBtn.className).toContain("bg-sky-500");
  });

  it("duration button is selected for tuplet child tones using their literal stored duration", () => {
    renderWithProviders(<LeftPanel spanRow={false} />, {
      appState: makeStateWithTupletTone(16),
    });

    const eighthBtn = screen.getByTitle("Eighth");
    expect(eighthBtn.className).toContain("bg-sky-500");
  });

  it("duration buttons are not visually disabled for tuplet child tones", () => {
    renderWithProviders(<LeftPanel spanRow={false} />, {
      appState: makeStateWithTupletTone(16),
    });

    const quarterBtn = screen.getByTitle("Quarter");
    expect(quarterBtn.className).not.toContain("pointer-events-none");
  });

  it("accidental button is selected when tone accidental matches", () => {
    renderWithProviders(<LeftPanel spanRow={false} />, {
      appState: makeStateWithTone(16, Accidental.Sharp),
    });
    const sharpBtn = screen.getByTitle("Sharp");
    expect(sharpBtn.className).toContain("bg-sky-500");
  });

  it("does not call setDuration when no tone selected (disabled button)", () => {
    renderWithProviders(<LeftPanel spanRow={false} />);
    // buttons are disabled — onClick guard prevents calls
    fireEvent.click(screen.getByTitle("Quarter"));
    expect(vi.mocked(actions.score.tone.setDuration)).not.toHaveBeenCalled();
  });
});
