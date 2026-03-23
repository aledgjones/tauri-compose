import { describe, it, expect, vi, beforeEach } from "vitest";
import { fireEvent, screen } from "@testing-library/react";
import { renderWithProviders } from "../../../../../services/test-utils";
import {
  Instrument,
  InstrumentType,
  ResolvedPlaybackRoute,
  ResolvedPlaybackRouteStatus,
} from "../../../../../services/types";
import { PlaybackStatusPanel } from "./playback-status-panel";

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
  target: null,
  address: null,
};

const unavailableRoute: ResolvedPlaybackRoute = {
  status: ResolvedPlaybackRouteStatus.Unrouted,
  slot_id: null,
  slot_target: null,
  effective_plugin_id: null,
  target: null,
  address: null,
};

describe("PlaybackStatusPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders selected instrument playback context when ready", () => {
    renderWithProviders(
      <PlaybackStatusPanel instrument={instrument} route={readyRoute} />,
      {
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

    expect(screen.getByText("Selected instrument")).toBeInTheDocument();
    expect(screen.getByText("Violin")).toBeInTheDocument();
    expect(
      screen.getByRole("status", { name: "Playback status: Ready" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Grand Piano")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /focus row route/i }),
    ).toBeNull();
  });

  it("uses one primary in-score recovery handoff for non-ready instruments", () => {
    const routeRow = document.createElement("div");
    routeRow.tabIndex = -1;
    routeRow.scrollIntoView = vi.fn();
    routeRow.setAttribute("data-route-focus-for", "5");
    document.body.appendChild(routeRow);

    renderWithProviders(
      <PlaybackStatusPanel instrument={instrument} route={unavailableRoute} />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Focus route summary for Violin" }),
    );

    expect(routeRow.scrollIntoView).toHaveBeenCalled();
    expect(routeRow).toHaveFocus();

    routeRow.remove();
  });
});
