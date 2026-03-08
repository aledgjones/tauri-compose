import { describe, expect, it } from "vitest";
import {
  renderWithProviders,
  createAppState,
  screen,
} from "../../../../../services/test-utils";
import { NoteDuration } from "../../../../../services/types";
import { PropertiesPanel } from "./properties-panel";

const FLOW_ID = 9;
const MASTER_ID = 10;
const TEMPO_ID = 44;

const buildTempoState = () =>
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
    expect(screen.getByTestId("tempo-properties-section")).toBeInTheDocument();
    expect(screen.getByText("Metronome")).toBeInTheDocument();
  });
});
