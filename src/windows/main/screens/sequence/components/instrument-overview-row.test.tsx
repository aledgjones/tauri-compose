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
    openExpressionMapWindow: vi.fn().mockResolvedValue(undefined),
    pluginInventory: {
      openEditorHost: vi.fn().mockResolvedValue(undefined),
    },
    ui: {
      sequencer: {
        setInstrument: vi.fn().mockResolvedValue(undefined),
      },
    },
    score: {
      instrument: {
        assignSlotRoute: vi.fn().mockResolvedValue(undefined),
        setExpressionMap: vi.fn().mockResolvedValue(undefined),
        unassignRoute: vi.fn().mockResolvedValue(undefined),
        setSolo: vi.fn().mockResolvedValue(undefined),
        setMute: vi.fn().mockResolvedValue(undefined),
      },
    },
  },
}));

import { actions } from "../../../../../services/actions";

const supportedMixerControls = {
  volume: true,
  pan: true,
  mute: true,
  solo: true,
};

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
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(actions.pluginInventory.openEditorHost).mockResolvedValue(
      undefined,
    );
  });

  const renderRow = (resolvedAssignment = {}) => {
    return renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={false}
        isRoutingExpanded={true}
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
              supported_mix_controls: {
                volume: false,
                pan: false,
                mute: false,
                solo: false,
              },
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
        expressionMapLibraryState: {
          order: ["legato", "staccato"],
          by_id: {
            legato: {
              id: "legato",
              name: "Legato",
              creator: "",
              version: 1,
              default_actions: [{ id: "legato-default-action", type: "no_op" }],
              rules: [],
            },
            staccato: {
              id: "staccato",
              name: "Staccato",
              creator: "",
              version: 1,
              default_actions: [
                { id: "staccato-default-action", type: "no_op" },
              ],
              rules: [],
            },
          },
        },
      },
    );
  };

  it("renders the instrument long name", () => {
    renderRow();
    expect(screen.getByText("Violin")).toBeInTheDocument();
  });

  it("always shows the inline routing controls without a separate expander", () => {
    renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={false}
        isRoutingExpanded={false}
      />,
    );

    expect(screen.getByRole("button", { name: /^--/i })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Expression Map: --/i }),
    ).toBeInTheDocument();
  });

  it("keeps unrouted instruments visually neutral in the route control", () => {
    renderRow();

    const routeButton = screen.getByRole("button", { name: /^--/i });

    expect(
      within(routeButton).queryByLabelText("Playback needs attention"),
    ).toBeNull();
    expect(
      within(routeButton).queryByTitle(/Assign a playback route/i),
    ).toBeNull();
  });

  it("does not render a playback warning when the backend route is ready", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      supported_mix_controls: supportedMixerControls,
    });

    const routeButton = screen.getByRole("button", {
      name: /1\. Grand Piano/i,
    });

    expect(
      within(routeButton).queryByLabelText("Playback needs attention"),
    ).toBeNull();
  });

  it("renders a plugin UI button beside the route picker for supported assigned plugins", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
      supported_mix_controls: supportedMixerControls,
    });

    expect(
      screen.getByRole("button", { name: "Open plugin UI for Grand Piano" }),
    ).toBeInTheDocument();
  });

  it("renders the assigned expression map name in the compact row control", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
      supported_mix_controls: supportedMixerControls,
      expression_map: undefined,
    });

    expect(
      screen.getByRole("button", { name: /Expression Map: --/i }),
    ).toBeInTheDocument();
  });

  it("shows the resolved assigned expression map name when present", () => {
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
            ],
          },
          resolved_playback_route_by_instrument_id: {
            5: {
              status: ResolvedPlaybackRouteStatus.Ready,
              slot_id: 7,
              slot_target: { plugin_id: "vst3:grand-piano" },
              effective_plugin_id: "vst3:grand-piano",
              target: { mode: 0, index: null },
              address: { mode: 0, index: null },
              supported_mix_controls: supportedMixerControls,
            },
          },
          resolved_expression_map_by_instrument_id: {
            5: {
              status: 1,
              expression_map_id: "legato",
              expression_map_name: "Legato",
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
        expressionMapLibraryState: {
          order: ["legato"],
          by_id: {
            legato: {
              id: "legato",
              name: "Legato",
              creator: "",
              version: 1,
              default_actions: [{ id: "legato-default-action", type: "no_op" }],
              rules: [],
            },
          },
        },
      },
    );

    expect(
      screen.getByRole("button", { name: /Expression Map: Legato/i }),
    ).toBeInTheDocument();
  });

  it("assigns an expression map from the row dropdown", async () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
      supported_mix_controls: supportedMixerControls,
    });

    fireEvent.click(
      screen.getByRole("button", { name: /Expression Map: --/i }),
    );
    fireEvent.click(screen.getByRole("option", { name: "Staccato" }));

    await waitFor(() => {
      expect(actions.score.instrument.setExpressionMap).toHaveBeenCalledWith(
        5,
        "staccato",
      );
    });
  });

  it("clears the assigned expression map back to the truthful unassigned state", async () => {
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
            ],
          },
          resolved_playback_route_by_instrument_id: {
            5: {
              status: ResolvedPlaybackRouteStatus.Ready,
              slot_id: 7,
              slot_target: { plugin_id: "vst3:grand-piano" },
              effective_plugin_id: "vst3:grand-piano",
              target: { mode: 0, index: null },
              address: { mode: 0, index: null },
              supported_mix_controls: supportedMixerControls,
            },
          },
          resolved_expression_map_by_instrument_id: {
            5: {
              status: 1,
              expression_map_id: "legato",
              expression_map_name: "Legato",
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
        expressionMapLibraryState: {
          order: ["legato"],
          by_id: {
            legato: {
              id: "legato",
              name: "Legato",
              creator: "",
              version: 1,
              default_actions: [{ id: "legato-default-action", type: "no_op" }],
              rules: [],
            },
          },
        },
      },
    );

    fireEvent.click(
      screen.getByRole("button", { name: /Expression Map: Legato/i }),
    );
    fireEvent.click(screen.getByRole("option", { name: "--" }));

    await waitFor(() => {
      expect(actions.score.instrument.setExpressionMap).toHaveBeenCalledWith(
        5,
        null,
      );
    });
  });

  it("disables the expression map edit button when no map is assigned", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
      supported_mix_controls: supportedMixerControls,
    });

    expect(
      screen.getByRole("button", { name: "Open expression map workflow" }),
    ).toBeDisabled();
  });

  it("opens the selected expression map shell from the instrument overview row", async () => {
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
            ],
          },
          resolved_playback_route_by_instrument_id: {
            5: {
              status: ResolvedPlaybackRouteStatus.Ready,
              slot_id: 7,
              slot_target: { plugin_id: "vst3:grand-piano" },
              effective_plugin_id: "vst3:grand-piano",
              target: { mode: 0, index: null },
              address: { mode: 0, index: null },
              supported_mix_controls: supportedMixerControls,
            },
          },
          resolved_expression_map_by_instrument_id: {
            5: {
              status: 1,
              expression_map_id: "legato",
              expression_map_name: "Legato",
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
        expressionMapLibraryState: {
          order: ["legato"],
          by_id: {
            legato: {
              id: "legato",
              name: "Legato",
              creator: "",
              version: 1,
              default_actions: [{ id: "legato-default-action", type: "no_op" }],
              rules: [],
            },
          },
        },
      },
    );

    fireEvent.click(
      screen.getByRole("button", {
        name: "Open expression map workflow for Legato",
      }),
    );

    await waitFor(() => {
      expect(actions.openExpressionMapWindow).toHaveBeenCalledWith("legato");
    });
  });

  it("opens the assigned plugin UI from the instrument overview row", async () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
      supported_mix_controls: supportedMixerControls,
    });

    fireEvent.click(
      screen.getByRole("button", { name: "Open plugin UI for Grand Piano" }),
    );

    await waitFor(() => {
      expect(actions.pluginInventory.openEditorHost).toHaveBeenCalledWith(
        "vst3:grand-piano",
        { slot_id: 7 },
      );
    });
  });

  it("hides the plugin UI button for blocked editor-host targets", () => {
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
            ],
          },
          resolved_playback_route_by_instrument_id: {
            5: {
              status: ResolvedPlaybackRouteStatus.Ready,
              slot_id: 7,
              slot_target: { plugin_id: "vst3:grand-piano" },
              effective_plugin_id: "vst3:grand-piano",
              target: { mode: 0, index: null },
              address: { mode: 0, index: null },
              supported_mix_controls: supportedMixerControls,
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
              editor_host_validation: {
                status: "blocked",
                phase: "policy_gate",
                message: "unsupported",
                capability_evidence: null,
              },
            },
          ],
        },
      },
    );

    expect(
      screen.getByRole("button", { name: "Open plugin UI for Grand Piano" }),
    ).toBeDisabled();
  });

  it("renders one compact inline warning when the backend route requires recovery", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.NeedsAttention,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      supported_mix_controls: {
        volume: false,
        pan: false,
        mute: false,
        solo: false,
      },
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

  it("renders both supported solo and mute controls beside the route picker", async () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
      supported_mix_controls: supportedMixerControls,
    });

    const routeButton = screen.getByRole("button", {
      name: /1\. Grand Piano/i,
    });
    const soloButton = screen.getByRole("button", { name: "Solo Violin" });
    const muteButton = screen.getByRole("button", { name: "Mute Violin" });

    expect(routeButton).toBeInTheDocument();
    expect(soloButton).toHaveAttribute("aria-pressed", "false");
    expect(muteButton).toHaveAttribute("aria-pressed", "false");
    fireEvent.click(muteButton);

    await waitFor(() => {
      expect(actions.score.instrument.setMute).toHaveBeenCalledWith(5, true);
    });
  });

  it("keeps solo/mute controls visible but disabled when the row is disabled", () => {
    const { container } = renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={true}
        isRoutingExpanded={true}
      />,
    );
    expect(container.querySelector(".opacity-50")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Solo Violin" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Mute Violin" })).toBeDisabled();
  });

  it("renders the current rack slot and route target summary", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      supported_mix_controls: supportedMixerControls,
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
      supported_mix_controls: supportedMixerControls,
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
        isRoutingExpanded={true}
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
              status: ResolvedPlaybackRouteStatus.NeedsAttention,
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
      within(routeButton).queryByLabelText("Playback needs attention"),
    ).toBeNull();
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

  it("keeps the score-context route picker usable after a missing slot normalizes to unrouted", async () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Unrouted,
      slot_id: null,
      slot_target: null,
      effective_plugin_id: null,
      target: null,
      address: null,
    });

    const routeButton = screen.getByRole("button", { name: /^--/i });

    expect(
      within(routeButton).queryByLabelText("Playback needs attention"),
    ).toBeNull();

    fireEvent.click(routeButton);
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
        isRoutingExpanded={true}
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
        isRoutingExpanded={true}
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

  it("changes an existing route to a different rack slot through the same picker", async () => {
    renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={false}
        isRoutingExpanded={true}
      />,
      {
        appState: {
          score: {
            playback: {
              rack: {
                order: [7, 8],
                by_id: {
                  7: {
                    id: 7,
                    target: { plugin_id: "vst3:grand-piano" },
                  },
                  8: {
                    id: 8,
                    target: { plugin_id: "vst3:string-ensemble" },
                  },
                },
              },
            },
          } as never,
          route_target_options_by_slot_id: {
            7: [
              {
                key: "plugin-root-7",
                label: "Plugin output",
                target: { mode: 0, index: null },
                address: { mode: 0, index: null },
              },
            ],
            8: [
              {
                key: "plugin-root-8",
                label: "Plugin output",
                target: { mode: 0, index: null },
                address: { mode: 0, index: null },
              },
            ],
          },
          resolved_playback_route_by_instrument_id: {
            5: {
              status: ResolvedPlaybackRouteStatus.Ready,
              slot_id: 7,
              slot_target: { plugin_id: "vst3:grand-piano" },
              effective_plugin_id: "vst3:grand-piano",
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
            {
              id: "vst3:string-ensemble",
              name: "String Ensemble",
              bundle_path: "/Library/Audio/Plug-Ins/VST3/String Ensemble.vst3",
              binary_path:
                "/Library/Audio/Plug-Ins/VST3/String Ensemble.vst3/Contents/MacOS/String Ensemble",
              source_root: "/Library/Audio/Plug-Ins/VST3",
              availability: "available",
              validation_failure: null,
              editor_host_validation: null,
            },
          ],
        },
      },
    );

    fireEvent.click(screen.getByRole("button", { name: /1\. Grand Piano/i }));
    fireEvent.click(screen.getByRole("option", { name: "2. String Ensemble" }));

    await waitFor(() => {
      expect(actions.ui.sequencer.setInstrument).toHaveBeenCalledWith(5);
      expect(actions.score.instrument.assignSlotRoute).toHaveBeenCalledWith(
        5,
        8,
        { mode: 0, index: null },
        { mode: 0, index: null },
      );
    });
  });

  it("keeps repair in the current route picker when an assigned plugin becomes unavailable", async () => {
    renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={false}
        isRoutingExpanded={true}
      />,
      {
        appState: {
          score: {
            playback: {
              rack: {
                order: [7, 8],
                by_id: {
                  7: {
                    id: 7,
                    target: { plugin_id: "vst3:grand-piano" },
                  },
                  8: {
                    id: 8,
                    target: { plugin_id: "vst3:string-ensemble" },
                  },
                },
              },
            },
          } as never,
          route_target_options_by_slot_id: {
            7: [],
            8: [
              {
                key: "plugin-root-8",
                label: "Plugin output",
                target: { mode: 0, index: null },
                address: { mode: 0, index: null },
              },
            ],
          },
          resolved_playback_route_by_instrument_id: {
            5: {
              status: ResolvedPlaybackRouteStatus.NeedsAttention,
              slot_id: 7,
              slot_target: { plugin_id: "vst3:grand-piano" },
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
              availability: "unavailable",
              validation_failure: {
                kind: "missing_bundle_binary",
                message: "missing",
              },
              editor_host_validation: null,
            },
            {
              id: "vst3:string-ensemble",
              name: "String Ensemble",
              bundle_path: "/Library/Audio/Plug-Ins/VST3/String Ensemble.vst3",
              binary_path:
                "/Library/Audio/Plug-Ins/VST3/String Ensemble.vst3/Contents/MacOS/String Ensemble",
              source_root: "/Library/Audio/Plug-Ins/VST3",
              availability: "available",
              validation_failure: null,
              editor_host_validation: null,
            },
          ],
        },
      },
    );

    const routeButton = screen.getByRole("button", {
      name: /1\. Grand Piano/i,
    });

    expect(
      within(routeButton).getByLabelText("Playback needs attention"),
    ).toBeInTheDocument();

    fireEvent.click(routeButton);
    fireEvent.click(screen.getByRole("option", { name: "2. String Ensemble" }));

    await waitFor(() => {
      expect(actions.ui.sequencer.setInstrument).toHaveBeenCalledWith(5);
      expect(actions.score.instrument.assignSlotRoute).toHaveBeenCalledWith(
        5,
        8,
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

  it("keeps the row quiet when direct overview routing fails", async () => {
    vi.mocked(actions.score.instrument.assignSlotRoute).mockRejectedValueOnce(
      new Error("Unsupported route target"),
    );

    renderRow();

    fireEvent.click(screen.getByRole("button", { name: /^--/i }));
    fireEvent.click(screen.getByRole("option", { name: "1. Grand Piano" }));

    await waitFor(() => {
      expect(actions.score.instrument.assignSlotRoute).toHaveBeenCalledWith(
        5,
        7,
        { mode: 0, index: null },
        { mode: 0, index: null },
      );
    });

    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("shows a recoverable error when a slot requires choosing between multiple targets", async () => {
    renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={false}
        isRoutingExpanded={true}
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

    const unsupportedOption = screen.getByRole("option", {
      name: "1. Grand Piano",
    });

    expect(unsupportedOption).toBeDisabled();

    fireEvent.click(unsupportedOption);

    expect(actions.score.instrument.assignSlotRoute).not.toHaveBeenCalled();
    expect(screen.queryByText(/Route update failed:/i)).not.toBeInTheDocument();
  });

  it("disables a non-empty slot when the backend emits no route choices for it", async () => {
    renderWithProviders(
      <InstrumentOverviewRow
        instrument={mockInstrument}
        color="#ff0000"
        isDisabled={false}
        isRoutingExpanded={true}
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

    const unroutableOption = screen.getByRole("option", {
      name: "1. Grand Piano",
    });

    expect(unroutableOption).toBeDisabled();

    fireEvent.click(unroutableOption);

    expect(actions.score.instrument.assignSlotRoute).not.toHaveBeenCalled();
    expect(screen.queryByText(/Route update failed:/i)).not.toBeInTheDocument();
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
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
      supported_mix_controls: supportedMixerControls,
    });

    const soloButton = screen.getByRole("button", { name: "Solo Violin" });

    fireEvent.click(soloButton);
    expect(actions.score.instrument.setSolo).toHaveBeenCalledWith(5, true);
  });

  it("calls setMute when mute button clicked", () => {
    renderRow({
      status: ResolvedPlaybackRouteStatus.Ready,
      slot_id: 7,
      slot_target: { plugin_id: "vst3:grand-piano" },
      effective_plugin_id: "vst3:grand-piano",
      target: { mode: 0, index: null },
      address: { mode: 0, index: null },
      supported_mix_controls: supportedMixerControls,
    });
    const muteButton = screen.getByRole("button", { name: "Mute Violin" });

    fireEvent.click(muteButton);
    expect(actions.score.instrument.setMute).toHaveBeenCalledWith(5, true);
  });

  it("exposes pressed state for selected instrument mix toggles", () => {
    renderWithProviders(
      <InstrumentOverviewRow
        instrument={{ ...mockInstrument, solo: true, mute: true }}
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
              status: ResolvedPlaybackRouteStatus.Ready,
              slot_id: 7,
              slot_target: { plugin_id: "vst3:grand-piano" },
              effective_plugin_id: "vst3:grand-piano",
              target: { mode: 0, index: null },
              address: { mode: 0, index: null },
              supported_mix_controls: supportedMixerControls,
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

    expect(screen.getByRole("button", { name: "Solo Violin" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(screen.getByRole("button", { name: "Mute Violin" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });

  it("renders instrument count when count is set", () => {
    const withCount: Instrument = { ...mockInstrument, count: "I" };
    renderWithProviders(
      <InstrumentOverviewRow
        instrument={withCount}
        color="#ff0000"
        isDisabled={false}
        isRoutingExpanded={true}
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
        isRoutingExpanded={true}
      />,
    );
    expect(container.firstChild).toHaveClass("opacity-50");
  });
});
