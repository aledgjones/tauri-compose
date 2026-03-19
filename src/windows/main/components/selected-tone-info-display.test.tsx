import { describe, it, expect } from "vitest";
import { screen } from "@testing-library/react";
import { SelectedToneInfoDisplay } from "./selected-tone-info-display";
import {
  renderWithProviders,
  createAppState,
} from "../../../services/test-utils";
import { Accidental, PlayerType } from "../../../services/types";

const FLOW_ID = 5;
const SECOND_FLOW_ID = 6;
const INSTRUMENT_ID = 10;
const STAVE_ID = 100;
const SECOND_STAVE_ID = 101;
const TRACK_ID = 300;
const SECOND_TRACK_ID = 301;
const TONE_ID = 999;
const SECOND_TONE_ID = 1000;

const buildStateWithTone = () =>
  createAppState({
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
            assigned_plugin_id: null,
            volume: 100,
            pan: 0,
            mute: false,
            solo: false,
          },
        },
      },
      flows: {
        order: [FLOW_ID],
        by_id: {
          [FLOW_ID]: {
            id: FLOW_ID,
            title: "Flow 1",
            player_ids: [1],
            length: 512,
            master_id: 200,
            staves: {
              by_id: {
                [STAVE_ID]: {
                  id: STAVE_ID,
                  lines: [1, 1, 1, 1, 1],
                  master_id: 200,
                  track_ids: [TRACK_ID],
                },
              },
            },
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

describe("SelectedToneInfoDisplay", () => {
  it("renders 'No selection' when no tone is selected", () => {
    renderWithProviders(<SelectedToneInfoDisplay />);
    expect(screen.getByText("No selection")).toBeInTheDocument();
  });

  it("renders instrument name when tone is selected", () => {
    renderWithProviders(<SelectedToneInfoDisplay />, {
      appState: buildStateWithTone(),
    });
    expect(screen.getByText(/Piano/)).toBeInTheDocument();
  });

  it("renders track label when tone is selected", () => {
    renderWithProviders(<SelectedToneInfoDisplay />, {
      appState: buildStateWithTone(),
    });
    expect(screen.getByText(/voice/)).toBeInTheDocument();
  });

  it("renders the existing summary fields for a selection in a later visible flow", () => {
    const appState = buildStateWithTone();
    appState.score.instruments.by_id[INSTRUMENT_ID].stave_ids = [
      STAVE_ID,
      SECOND_STAVE_ID,
    ];
    appState.score.flows.order = [FLOW_ID, SECOND_FLOW_ID];
    appState.score.flows.by_id[SECOND_FLOW_ID] = {
      id: SECOND_FLOW_ID,
      title: "Flow 2",
      player_ids: [1],
      length: 512,
      master_id: 201,
      staves: {
        by_id: {
          [SECOND_STAVE_ID]: {
            id: SECOND_STAVE_ID,
            lines: [1, 1, 1, 1, 1],
            master_id: 201,
            track_ids: [SECOND_TRACK_ID],
          },
        },
      },
    } as never;
    appState.score.tracks.by_id[SECOND_TRACK_ID] = {
      id: SECOND_TRACK_ID,
      entries: {
        by_tick: { [96]: [SECOND_TONE_ID] },
        by_id: {
          [SECOND_TONE_ID]: {
            id: SECOND_TONE_ID,
            tick: 96,
            type: "Tone",
            duration: 32,
            pitch: { int: 62, accidental: Accidental.Natural },
            velocity: { int: 100 },
            articulations: [],
          },
        },
      },
    } as never;
    appState.ui.sequencer.selection = {
      type: "Tone",
      flow_id: SECOND_FLOW_ID,
      instrument_id: INSTRUMENT_ID,
      stave_id: SECOND_STAVE_ID,
      track_id: SECOND_TRACK_ID,
      tone_id: SECOND_TONE_ID,
    };

    renderWithProviders(<SelectedToneInfoDisplay />, {
      appState,
    });

    expect(screen.getByText(/Piano/)).toBeInTheDocument();
    expect(
      screen.getByText(/Staff \(B\), Up-stem voice 1/),
    ).toBeInTheDocument();
  });

  it("renders the shared summary for a tuplet-owned tone selection", () => {
    const appState = buildStateWithTone();
    appState.score.tracks.by_id[TRACK_ID] = {
      id: TRACK_ID,
      entries: {
        by_tick: { 64: [700] },
        by_id: {
          700: {
            id: 700,
            tick: 64,
            type: "Tuplet",
            note_count: 3,
            in_space_of: 2,
            base_note_value: 8,
            groupings: [3],
            entries: {
              by_tick: { 0: [701] },
              by_id: {
                701: {
                  id: 701,
                  tick: 0,
                  type: "Tone",
                  duration: 16,
                  pitch: { int: 62, accidental: Accidental.Natural },
                  velocity: { int: 100 },
                  articulations: [],
                },
              },
            },
          },
        },
      },
    } as never;
    appState.ui.sequencer.selection = {
      type: "Tone",
      flow_id: FLOW_ID,
      instrument_id: INSTRUMENT_ID,
      stave_id: STAVE_ID,
      track_id: TRACK_ID,
      tone_id: 701,
      tuplet_id: 700,
    };

    renderWithProviders(<SelectedToneInfoDisplay />, {
      appState,
    });

    expect(screen.getByText(/Piano/)).toBeInTheDocument();
    expect(screen.getByText(/voice/)).toBeInTheDocument();
  });
});
