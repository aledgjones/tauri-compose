import { describe, it, expect } from "vitest";
import { renderWithProviders } from "../../../../../services/test-utils";
import {
  Instrument,
  InstrumentType,
  ResolvedPlaybackRoute,
  ResolvedPlaybackRouteStatus,
} from "../../../../../services/types";
import { screen } from "@testing-library/react";
import { PlaybackRackPanel } from "./playback-rack-panel";

const instrument: Instrument = {
  id: 5,
  definition_id: "violin",
  instrument_type: InstrumentType.Melodic,
  long_name: "Violin",
  short_name: "Vln.",
  stave_ids: [],
  count: null,
  playback_route: null,
  volume: 100,
  pan: 64,
  mute: false,
  solo: false,
};

const readyRoute: ResolvedPlaybackRoute = {
  status: ResolvedPlaybackRouteStatus.Ready,
  slot_id: 9,
  slot_target: { plugin_id: "vst3:grand-piano" },
  effective_plugin_id: "vst3:grand-piano",
  target: { mode: 0, index: null },
  address: { mode: 0, index: null },
};

describe("PlaybackRackPanel", () => {
  it("renders current rack slots including empty slots", () => {
    renderWithProviders(
      <PlaybackRackPanel instrument={instrument} route={readyRoute} />,
      {
        appState: {
          score: {
            playback: {
              rack: {
                order: [9, 10],
                by_id: {
                  9: {
                    id: 9,
                    target: { plugin_id: "vst3:grand-piano" },
                  },
                  10: {
                    id: 10,
                    target: null,
                  },
                },
              },
            },
          } as never,
        },
        pluginInventoryState: {
          entries: [
            {
              id: "vst3:grand-piano",
              name: "Grand Piano",
              bundle_path: "/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3",
              binary_path:
                "/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3/Contents/MacOS/Grand Piano",
              source_root: "/Library/Audio/Plug-Ins/VST3",
              availability: "available",
              validation_failure: null,
            },
          ],
        },
      },
    );

    expect(screen.getByText("Current rack")).toBeInTheDocument();
    expect(screen.getByText("1. Grand Piano")).toBeInTheDocument();
    expect(screen.getByText("2. Unassigned")).toBeInTheDocument();
    expect(screen.getByText("Focused instrument route")).toBeInTheDocument();
    expect(screen.queryByLabelText("Playback needs attention")).toBeNull();
  });

  it("renders empty-state guidance when the rack has no slots yet", () => {
    renderWithProviders(
      <PlaybackRackPanel instrument={instrument} route={null} />,
    );

    expect(
      screen.getByText("Current rack has no score slots yet."),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        /Plugin Manager remains the app-scoped inventory browser\./i,
      ),
    ).toBeInTheDocument();
  });

  it("shows a missing-slot warning when the route points at a removed slot", () => {
    renderWithProviders(
      <PlaybackRackPanel instrument={instrument} route={readyRoute} />,
      {
        appState: {
          score: {
            playback: {
              rack: {
                order: [],
                by_id: {},
              },
            },
          } as never,
        },
        pluginInventoryState: {
          entries: [
            {
              id: "vst3:grand-piano",
              name: "Grand Piano",
              bundle_path: "/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3",
              binary_path:
                "/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3/Contents/MacOS/Grand Piano",
              source_root: "/Library/Audio/Plug-Ins/VST3",
              availability: "available",
              validation_failure: null,
            },
          ],
        },
      },
    );

    expect(screen.getByText("Missing routed slot")).toBeInTheDocument();
    expect(
      screen.getByLabelText("Playback needs attention"),
    ).toBeInTheDocument();
  });
});
