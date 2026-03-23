import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { InstrumentOverviewRow } from "./instrument-overview-row";
import { renderWithProviders } from "../../../../../services/test-utils";
import {
  BottomPanelTab,
  Instrument,
  InstrumentType,
  ResolvedPlaybackRouteStatus,
} from "../../../../../services/types";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    ui: {
      screens: {
        sequence: {
          bottomPanel: {
            setTab: vi.fn().mockResolvedValue(undefined),
          },
        },
      },
      sequencer: {
        setInstrument: vi.fn().mockResolvedValue(undefined),
      },
    },
    score: {
      instrument: {
        setSolo: vi.fn().mockResolvedValue(undefined),
        setMute: vi.fn().mockResolvedValue(undefined),
      },
    },
  },
}));

import { actions } from "../../../../../services/actions";

const mockInstrument: Instrument = {
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

describe("InstrumentOverviewRow", () => {
  beforeEach(() => vi.clearAllMocks());

  const renderRow = (resolvedAssignment = {}) => {
    return renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={false}
      />,
      {
        appState: {
          score: {
            playback: {
              rack: {
                order: [7],
                by_id: {
                  7: {
                    id: 7,
                    target: { plugin_id: "vst3:grand-piano" },
                  },
                },
              },
            },
          } as never,
          resolved_playback_route_by_instrument_id: {
            5: {
              status: ResolvedPlaybackRouteStatus.Unrouted,
              slot_id: null,
              slot_target: null,
              effective_plugin_id: null,
              target: null,
              address: null,
              ...resolvedAssignment,
            },
          },
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
  };

  it("renders the instrument long name", () => {
    renderRow();
    expect(screen.getByText("Violin")).toBeInTheDocument();
  });

  it("does not render a playback warning by default", () => {
    renderRow();

    expect(screen.queryByLabelText("Playback needs attention")).toBeNull();
  });

  it("does not render a playback warning when the backend route is ready", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
    });

    expect(screen.queryByLabelText("Playback needs attention")).toBeNull();
  });

  it("renders a needs-attention badge when the backend route requires recovery", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.NeedsAttention,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
    });

    expect(
      screen.getByLabelText("Playback needs attention"),
    ).toBeInTheDocument();
  });

  it("renders solo and mute buttons when not disabled", () => {
    renderRow();
    const buttons = screen.getAllByRole("button");
    // solo + mute buttons
    expect(buttons.length).toBeGreaterThanOrEqual(2);
  });

  it("hides solo/mute controls when disabled (applies opacity-50 class)", () => {
    const { container } = renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={true}
      />,
    );
    expect(container.querySelector(".opacity-50")).toBeInTheDocument();
  });

  it("renders the current rack slot and route target summary", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
    });

    expect(screen.getByText("Slot 1. Grand Piano")).toBeInTheDocument();
    expect(screen.getByText("Target Plugin output")).toBeInTheDocument();
  });

  it("renders a missing route summary by default", () => {
    renderRow();
    expect(screen.getByText("No routed slot yet")).toBeInTheDocument();
    expect(screen.getByText("No route target selected")).toBeInTheDocument();
  });

  it("uses one primary recovery action for non-ready routes", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.NeedsAttention,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
    });

    fireEvent.click(
      screen.getByRole("button", { name: "Open rack for Violin" }),
    );

    expect(actions.ui.sequencer.setInstrument).toHaveBeenCalledWith(5);
    expect(actions.ui.screens.sequence.bottomPanel.setTab).toHaveBeenCalledWith(
      BottomPanelTab.Instruments,
    );
  });

  it("treats a route pointing at a missing slot as needs attention", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 99,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
    });

    expect(
      screen.getByLabelText("Playback needs attention"),
    ).toBeInTheDocument();
    expect(screen.getByText("Missing slot 99")).toBeInTheDocument();
  });

  it("calls setSolo when solo button clicked", () => {
    renderRow();
    const [soloBtn] = screen.getAllByRole("button");
    fireEvent.click(soloBtn);
    expect(actions.score.instrument.setSolo).toHaveBeenCalledWith(5, true);
  });

  it("calls setMute when mute button clicked", () => {
    renderRow();
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[1]);
    expect(actions.score.instrument.setMute).toHaveBeenCalledWith(5, true);
  });

  it("renders instrument count when count is set", () => {
    const withCount: Instrument = { ...mockInstrument, count: "I" };
    renderWithProviders(
      <InstrumentOverviewRow
        instrument={withCount}
        color="#ff0000"
        isDisabled={false}
      />,
    );
    expect(screen.getByText("I")).toBeInTheDocument();
  });

  it("applies opacity when disabled", () => {
    const { container } = renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={true}
      />,
    );
    expect(container.firstChild).toHaveClass("opacity-50");
  });
});
