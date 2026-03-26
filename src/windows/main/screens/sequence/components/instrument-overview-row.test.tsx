import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent, waitFor } from "@testing-library/react";
import { InstrumentOverviewRow } from "./instrument-overview-row";
import { renderWithProviders } from "../../../../../services/test-utils";
import { within } from "@testing-library/dom";
import {
  Instrument,
  InstrumentType,
  ResolvedPlaybackRouteStatus,
} from "../../../../../services/types";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    ui: {
      sequencer: {
        setInstrument: vi.fn().mockResolvedValue(undefined),
      },
    },
    score: {
      instrument: {
        assignSlotRoute: vi.fn().mockResolvedValue(undefined),
        unassignRoute: vi.fn().mockResolvedValue(undefined),
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
          route_target_options_by_slot_id: {
            7: [
              {
                key: "plugin-root",
                label: "Plugin output",
                target: { mode: 0, index: null },
                address: { mode: 0, index: null },
              },
            ],
          },
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
              editor_host_validation: null,
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

  it("renders one compact inline playback warning when the instrument is unrouted", () => {
    renderRow();

    const routeButton = screen.getByRole("button", { name: /^--/i });

    expect(
      within(routeButton).getByLabelText("Playback needs attention"),
    ).toBeInTheDocument();
    expect(
      within(routeButton).getByTitle(
        "Assign a playback route to enable playback.",
      ),
    ).toBeInTheDocument();
  });

  it("does not render a playback warning when the backend route is ready", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
    });

    const routeButton = screen.getByRole("button", {
      name: /1\. Grand Piano/i,
    });

    expect(
      within(routeButton).queryByLabelText("Playback needs attention"),
    ).toBeNull();
  });

  it("renders one compact inline warning when the backend route requires recovery", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.NeedsAttention,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
    });

    const routeButton = screen.getByRole("button", {
      name: /1\. Grand Piano/i,
    });

    expect(
      within(routeButton).getByLabelText("Playback needs attention"),
    ).toBeInTheDocument();
    expect(
      within(routeButton).getByTitle(
        "Repair this playback route to enable playback.",
      ),
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

    expect(screen.queryByText("Slot 1. Grand Piano")).toBeNull();
    expect(screen.queryByText("Target: plugin output")).toBeNull();
  });

  it("renders the slot picker with the current slot selected", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
    });

    expect(
      screen.getByRole("button", { name: /1\. Grand Piano/i }),
    ).toBeInTheDocument();
  });

  it("renders a missing route summary by default", () => {
    renderRow();
    expect(screen.queryByText("No routed slot yet")).toBeNull();
    expect(screen.queryByText("Target: --")).toBeNull();
    expect(screen.getByRole("button", { name: /^--/i })).toBeInTheDocument();
  });

  it("shows an existing empty slot as a numbered selected slot", () => {
    renderWithProviders(
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
                    target: null,
                  },
                },
              },
            },
          } as never,
          route_target_options_by_slot_id: {
            7: [],
          },
          resolved_playback_route_by_instrument_id: {
            5: {
              status: ResolvedPlaybackRouteStatus.Unrouted,
              slot_id: 7,
              slot_target: null,
              effective_plugin_id: null,
              target: { mode: 0, index: null },
              address: { mode: 0, index: null },
            },
          },
        },
        pluginInventoryState: {
          entries: [],
        },
      },
    );

    const routeButton = screen.getByRole("button", { name: /1\. --/i });

    expect(routeButton).toBeInTheDocument();
    expect(
      within(routeButton).getByLabelText("Playback needs attention"),
    ).toBeInTheDocument();
  });

  it("keeps the compact warning on the route control instead of next to the instrument name", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.NeedsAttention,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
    });

    const instrumentName = screen.getByText("Violin").closest("div");
    const routeButton = screen.getByRole("button", {
      name: /1\. Grand Piano/i,
    });

    expect(
      instrumentName?.querySelector('[aria-label="Playback needs attention"]'),
    ).toBeNull();
    expect(
      within(routeButton).getByLabelText("Playback needs attention"),
    ).toBeInTheDocument();
  });

  it("does not render a separate open-rack action button", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.NeedsAttention,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
    });

    expect(screen.queryByRole("button", { name: /open rack/i })).toBeNull();
  });

  it("routes to a selected slot directly from the overview picker", async () => {
    renderRow();

    fireEvent.click(screen.getByRole("button", { name: /^--/i }));
    fireEvent.click(screen.getByRole("option", { name: "1. Grand Piano" }));

    await waitFor(() => {
      expect(actions.ui.sequencer.setInstrument).toHaveBeenCalledWith(5);
      expect(actions.score.instrument.assignSlotRoute).toHaveBeenCalledWith(
        5,
        7,
        { mode: 0, index: null },
        { mode: 0, index: null },
      );
    });
  });

  it("uses the backend-owned route choice when a slot exposes a non-root target", async () => {
    renderWithProviders(
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
          route_target_options_by_slot_id: {
            7: [
              {
                key: "plugin-part-2",
                label: "Plugin part 2",
                target: { mode: 1, index: 1 },
                address: { mode: 1, index: 1 },
              },
            ],
          },
          resolved_playback_route_by_instrument_id: {
            5: {
              status: ResolvedPlaybackRouteStatus.Unrouted,
              slot_id: null,
              slot_target: null,
              effective_plugin_id: null,
              target: null,
              address: null,
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
              editor_host_validation: null,
            },
          ],
        },
      },
    );

    fireEvent.click(screen.getByRole("button", { name: /^--/i }));
    fireEvent.click(screen.getByRole("option", { name: "1. Grand Piano" }));

    await waitFor(() => {
      expect(actions.score.instrument.assignSlotRoute).toHaveBeenCalledWith(
        5,
        7,
        { mode: 1, index: 1 },
        { mode: 1, index: 1 },
      );
    });
  });

  it("routes to an existing empty slot from the overview picker", async () => {
    renderWithProviders(
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
                    target: null,
                  },
                },
              },
            },
          } as never,
          route_target_options_by_slot_id: {
            7: [],
          },
          resolved_playback_route_by_instrument_id: {
            5: {
              status: ResolvedPlaybackRouteStatus.Unrouted,
              slot_id: null,
              slot_target: null,
              effective_plugin_id: null,
              target: null,
              address: null,
            },
          },
        },
        pluginInventoryState: {
          entries: [],
        },
      },
    );

    fireEvent.click(screen.getByRole("button", { name: /^--/i }));
    fireEvent.click(screen.getByRole("option", { name: "1. --" }));

    await waitFor(() => {
      expect(actions.ui.sequencer.setInstrument).toHaveBeenCalledWith(5);
      expect(actions.score.instrument.assignSlotRoute).toHaveBeenCalledWith(
        5,
        7,
        { mode: 0, index: null },
        { mode: 0, index: null },
      );
    });
  });

  it("can clear a route by switching the picker back to unrouted", async () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
    });

    fireEvent.click(screen.getByRole("button", { name: /1\. Grand Piano/i }));
    fireEvent.click(screen.getByRole("option", { name: "--" }));

    await waitFor(() => {
      expect(actions.score.instrument.unassignRoute).toHaveBeenCalledWith(5);
    });
  });

  it("shows a recoverable error when direct overview routing fails", async () => {
    vi.mocked(actions.score.instrument.assignSlotRoute).mockRejectedValueOnce(
      new Error("Unsupported route target"),
    );

    renderRow();

    fireEvent.click(screen.getByRole("button", { name: /^--/i }));
    fireEvent.click(screen.getByRole("option", { name: "1. Grand Piano" }));

    await waitFor(() => {
      expect(screen.getByText(/Route update failed:/i)).toHaveTextContent(
        "Route update failed: Unsupported route target",
      );
    });
  });

  it("shows a recoverable error when a slot requires choosing between multiple targets", async () => {
    renderWithProviders(
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
          route_target_options_by_slot_id: {
            7: [
              {
                key: "plugin-root",
                label: "Plugin output",
                target: { mode: 0, index: null },
                address: { mode: 0, index: null },
              },
              {
                key: "plugin-part-1",
                label: "Plugin part 1",
                target: { mode: 1, index: 0 },
                address: { mode: 1, index: 0 },
              },
            ],
          },
          resolved_playback_route_by_instrument_id: {
            5: {
              status: ResolvedPlaybackRouteStatus.Unrouted,
              slot_id: null,
              slot_target: null,
              effective_plugin_id: null,
              target: null,
              address: null,
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
              editor_host_validation: null,
            },
          ],
        },
      },
    );

    fireEvent.click(screen.getByRole("button", { name: /^--/i }));
    fireEvent.click(screen.getByRole("option", { name: "1. Grand Piano" }));

    await waitFor(() => {
      expect(screen.getByText(/Route update failed:/i)).toHaveTextContent(
        "Route update failed: Selected slot needs a specific route target that this picker cannot choose yet.",
      );
    });

    expect(actions.score.instrument.assignSlotRoute).not.toHaveBeenCalled();
  });

  it("refreshes the visible summary after backend route state updates", () => {
    const initial = renderRow();

    expect(screen.getByRole("button", { name: /^--/i })).toBeInTheDocument();

    initial.unmount();

    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
    });

    expect(
      screen.getByRole("button", { name: /1\. Grand Piano/i }),
    ).toBeInTheDocument();
  });

  it("refreshes the selected slot label after the slot plugin changes", () => {
    const initial = renderWithProviders(
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
                    target: null,
                  },
                },
              },
            },
          } as never,
          route_target_options_by_slot_id: {
            7: [],
          },
          resolved_playback_route_by_instrument_id: {
            5: {
              status: ResolvedPlaybackRouteStatus.Unrouted,
              slot_id: 7,
              slot_target: null,
              effective_plugin_id: null,
              target: { mode: 0, index: null },
              address: { mode: 0, index: null },
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
              editor_host_validation: null,
            },
          ],
        },
      },
    );

    expect(screen.getByRole("button", { name: /1\. --/i })).toBeInTheDocument();

    initial.unmount();

    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
    });

    expect(
      screen.getByRole("button", { name: /1\. Grand Piano/i }),
    ).toBeInTheDocument();
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
