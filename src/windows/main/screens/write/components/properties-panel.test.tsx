import { describe, expect, it } from "vitest";
import {
  renderWithProviders,
  createAppState,
  screen,
} from "../../../../../services/test-utils";
import {
  KeySignatureMode,
  NoteDuration,
  TimeSignatureDrawType,
} from "../../../../../services/types";
import { PropertiesPanel } from "./properties-panel";

const FLOW_ID = 9;
const MASTER_ID = 10;
const STAVE_ID = 11;
const STAVE_MASTER_ID = 12;
const TEMPO_ID = 44;
const TIME_SIGNATURE_ID = 45;
const KEY_SIGNATURE_ID = 46;
const CLEF_ID = 47;

const buildBaseScore = () => ({
  flows: {
    order: [FLOW_ID],
    by_id: {
      [FLOW_ID]: {
        id: FLOW_ID,
        title: "Flow",
        player_ids: [],
        length: 128,
        master_id: MASTER_ID,
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
      [MASTER_ID]: {
        id: MASTER_ID,
        entries: {
          by_tick: {
            64: [TEMPO_ID, TIME_SIGNATURE_ID, KEY_SIGNATURE_ID],
          },
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
            [TIME_SIGNATURE_ID]: {
              id: TIME_SIGNATURE_ID,
              tick: 64,
              type: "TimeSignature",
              hidden: false,
              beats: 4,
              beat_type: NoteDuration.Quarter,
              draw_type: TimeSignatureDrawType.Regular,
            },
            [KEY_SIGNATURE_ID]: {
              id: KEY_SIGNATURE_ID,
              tick: 64,
              type: "KeySignature",
              hidden: false,
              mode: KeySignatureMode.Major,
              offset: 2,
            },
          },
        },
      },
      [STAVE_MASTER_ID]: {
        id: STAVE_MASTER_ID,
        entries: {
          by_tick: { 64: [CLEF_ID] },
          by_id: {
            [CLEF_ID]: {
              id: CLEF_ID,
              tick: 64,
              type: "Clef",
              hidden: false,
              draw_as: 1,
              pitch: { int: 67, accidental: 0 },
              offset: 2,
            },
          },
        },
      },
    },
  },
});

const buildTempoState = () =>
  createAppState({
    score: buildBaseScore() as never,
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

describe("PropertiesPanel", () => {
  it("renders the no-selection empty state", () => {
    renderWithProviders(<PropertiesPanel />);
    expect(screen.getByTestId("properties-panel-empty")).toBeInTheDocument();
  });

  it("renders the unsupported-selection empty state", () => {
    const appState = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Rest",
            flow_id: 1,
            instrument_id: 2,
            stave_id: 3,
            track_id: 4,
            tick: 64,
          },
        },
      } as never,
    });

    renderWithProviders(<PropertiesPanel />, { appState });
    expect(
      screen.getByTestId("properties-panel-unsupported"),
    ).toBeInTheDocument();
  });

  it("routes tempo selections to the tempo properties section", () => {
    renderWithProviders(<PropertiesPanel />, { appState: buildTempoState() });
    expect(screen.getByText("Tempo")).toBeInTheDocument();
    expect(screen.getByText("Common")).toBeInTheDocument();
    expect(screen.getByText("Show metronome")).toBeInTheDocument();
  });

  it("routes time signature selections to the time signature properties section", () => {
    const appState = createAppState({
      score: buildBaseScore() as never,
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

    renderWithProviders(<PropertiesPanel />, { appState });
    expect(screen.getByText("Time signature")).toBeInTheDocument();
    expect(
      screen.getByRole("switch", { name: "Symbol form" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("switch", { name: "Hidden" })).toBeInTheDocument();
  });

  it("routes key signature selections to the key signature properties section", () => {
    const appState = createAppState({
      score: buildBaseScore() as never,
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

    renderWithProviders(<PropertiesPanel />, { appState });
    expect(screen.getByText("Common")).toBeInTheDocument();
    expect(screen.getByRole("switch", { name: "Hidden" })).toBeInTheDocument();
  });

  it("routes clef selections to the clef properties section", () => {
    const appState = createAppState({
      score: buildBaseScore() as never,
      ui: {
        sequencer: {
          selection: {
            type: "Clef",
            flow_id: FLOW_ID,
            stave_id: STAVE_ID,
            clef_id: CLEF_ID,
            tick: 64,
          },
        },
      } as never,
    });

    renderWithProviders(<PropertiesPanel />, { appState });
    expect(screen.getByText("Common")).toBeInTheDocument();
    expect(screen.getByRole("switch", { name: "Hidden" })).toBeInTheDocument();
  });
});
