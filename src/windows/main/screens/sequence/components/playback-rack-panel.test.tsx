import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderWithProviders } from "../../../../../services/test-utils";
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { PlaybackRackPanel } from "./playback-rack-panel";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    pluginInventory: {
      openEditorPoc: vi.fn().mockResolvedValue(undefined),
    },
    score: {
      playback: {
        slot: {
          assignPlugin: vi.fn().mockResolvedValue(undefined),
          unassignPlugin: vi.fn().mockResolvedValue(undefined),
        },
      },
    },
  },
}));

import { actions } from "../../../../../services/actions";

describe("PlaybackRackPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(actions.pluginInventory.openEditorPoc).mockResolvedValue(
      undefined,
    );
    vi.mocked(actions.score.playback.slot.assignPlugin).mockResolvedValue(
      undefined,
    );
    vi.mocked(actions.score.playback.slot.unassignPlugin).mockResolvedValue(
      undefined,
    );
  });

  it("renders current rack slots including empty slots", () => {
    renderWithProviders(<PlaybackRackPanel />, {
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
            editor_host_validation: null,
          },
        ],
      },
    });

    expect(screen.getByText("Slot 1")).toBeInTheDocument();
    expect(screen.getByText("Slot 2")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Open plugin UI for Grand Piano" }),
    ).toBeInTheDocument();
    expect(screen.queryByText("Focused instrument route")).toBeNull();
  });

  it("opens the assigned plugin UI from the slot card", async () => {
    renderWithProviders(<PlaybackRackPanel />, {
      appState: {
        score: {
          playback: {
            rack: {
              order: [9],
              by_id: {
                9: {
                  id: 9,
                  target: { plugin_id: "vst3:grand-piano" },
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
            editor_host_validation: null,
          },
        ],
      },
    });

    fireEvent.click(
      screen.getByRole("button", { name: "Open plugin UI for Grand Piano" }),
    );

    await waitFor(() => {
      expect(actions.pluginInventory.openEditorPoc).toHaveBeenCalledWith(
        "vst3:grand-piano",
        { slot_id: 9 },
      );
    });
  });

  it("surfaces a local error when opening the assigned plugin UI fails", async () => {
    vi.mocked(actions.pluginInventory.openEditorPoc).mockRejectedValueOnce(
      new Error("Attach failed"),
    );

    renderWithProviders(<PlaybackRackPanel />, {
      appState: {
        score: {
          playback: {
            rack: {
              order: [9],
              by_id: {
                9: {
                  id: 9,
                  target: { plugin_id: "vst3:grand-piano" },
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
            editor_host_validation: null,
          },
        ],
      },
    });

    fireEvent.click(
      screen.getByRole("button", { name: "Open plugin UI for Grand Piano" }),
    );

    expect(
      await screen.findByText("Plugin UI failed: Attach failed"),
    ).toBeInTheDocument();
  });

  it("renders empty-state guidance when the rack has no slots yet", () => {
    renderWithProviders(<PlaybackRackPanel />, {
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
    });

    expect(screen.getByText("No score slots yet.")).toBeInTheDocument();
    expect(
      screen.getByText(
        /Use the add button to create an empty slot, then choose an available plugin on that slot\./i,
      ),
    ).toBeInTheDocument();
  });

  it("filters slot plugin options to selectable plugins only", () => {
    renderWithProviders(<PlaybackRackPanel />, {
      appState: {
        score: {
          playback: {
            rack: {
              order: [12],
              by_id: {
                12: {
                  id: 12,
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
            editor_host_validation: null,
          },
          {
            id: "vst3:unsupported",
            name: "Unsupported Plugin",
            bundle_path: "/Library/Audio/Plug-Ins/VST3/Unsupported Plugin.vst3",
            binary_path:
              "/Library/Audio/Plug-Ins/VST3/Unsupported Plugin.vst3/Contents/MacOS/Unsupported Plugin",
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
          {
            id: "vst3:missing",
            name: "Missing Plugin",
            bundle_path: "/Library/Audio/Plug-Ins/VST3/Missing Plugin.vst3",
            binary_path: null,
            source_root: "/Library/Audio/Plug-Ins/VST3",
            availability: "unavailable",
            validation_failure: {
              kind: "missing_bundle_binary",
              message: "missing",
            },
            editor_host_validation: null,
          },
        ],
      },
    });

    fireEvent.click(screen.getByRole("button", { name: /^--$/i }));

    expect(screen.getByRole("option", { name: "--" })).toBeInTheDocument();
    expect(
      screen.getByRole("option", { name: "Grand Piano" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("option", { name: "Unsupported Plugin" }),
    ).toBeNull();
    expect(screen.queryByRole("option", { name: "Missing Plugin" })).toBeNull();
  });

  it("assigns a plugin to an existing slot from the slot-local picker", async () => {
    renderWithProviders(<PlaybackRackPanel />, {
      appState: {
        score: {
          playback: {
            rack: {
              order: [12],
              by_id: {
                12: {
                  id: 12,
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
            editor_host_validation: null,
          },
        ],
      },
    });

    fireEvent.click(screen.getByRole("button", { name: /^--$/i }));
    fireEvent.click(screen.getByRole("option", { name: "Grand Piano" }));

    await waitFor(() => {
      expect(actions.score.playback.slot.assignPlugin).toHaveBeenCalledWith(
        12,
        "vst3:grand-piano",
      );
    });
  });

  it("updates an already assigned slot from the slot-local picker", async () => {
    renderWithProviders(<PlaybackRackPanel />, {
      appState: {
        score: {
          playback: {
            rack: {
              order: [12],
              by_id: {
                12: {
                  id: 12,
                  target: { plugin_id: "vst3:grand-piano" },
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
            editor_host_validation: null,
          },
          {
            id: "vst3:warm-strings",
            name: "Warm Strings",
            bundle_path: "/Library/Audio/Plug-Ins/VST3/Warm Strings.vst3",
            binary_path:
              "/Library/Audio/Plug-Ins/VST3/Warm Strings.vst3/Contents/MacOS/Warm Strings",
            source_root: "/Library/Audio/Plug-Ins/VST3",
            availability: "available",
            validation_failure: null,
            editor_host_validation: null,
          },
        ],
      },
    });

    fireEvent.click(screen.getByRole("button", { name: /^grand piano$/i }));
    fireEvent.click(screen.getByRole("option", { name: "Warm Strings" }));

    await waitFor(() => {
      expect(actions.score.playback.slot.assignPlugin).toHaveBeenCalledWith(
        12,
        "vst3:warm-strings",
      );
    });
  });

  it("clears an assigned plugin when -- is selected", async () => {
    renderWithProviders(<PlaybackRackPanel />, {
      appState: {
        score: {
          playback: {
            rack: {
              order: [12],
              by_id: {
                12: {
                  id: 12,
                  target: { plugin_id: "vst3:grand-piano" },
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
            editor_host_validation: null,
          },
        ],
      },
    });

    fireEvent.click(screen.getByRole("button", { name: /^grand piano$/i }));
    fireEvent.click(screen.getByRole("option", { name: "--" }));

    await waitFor(() => {
      expect(actions.score.playback.slot.unassignPlugin).toHaveBeenCalledWith(
        12,
      );
    });
  });

  it("keeps the stored selection and shows an error when assignment fails", async () => {
    vi.mocked(actions.score.playback.slot.assignPlugin).mockRejectedValueOnce(
      new Error("Assignment failed"),
    );

    renderWithProviders(<PlaybackRackPanel />, {
      appState: {
        score: {
          playback: {
            rack: {
              order: [12],
              by_id: {
                12: {
                  id: 12,
                  target: { plugin_id: "vst3:grand-piano" },
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
            editor_host_validation: null,
          },
          {
            id: "vst3:warm-strings",
            name: "Warm Strings",
            bundle_path: "/Library/Audio/Plug-Ins/VST3/Warm Strings.vst3",
            binary_path:
              "/Library/Audio/Plug-Ins/VST3/Warm Strings.vst3/Contents/MacOS/Warm Strings",
            source_root: "/Library/Audio/Plug-Ins/VST3",
            availability: "available",
            validation_failure: null,
            editor_host_validation: null,
          },
        ],
      },
    });

    fireEvent.click(screen.getByRole("button", { name: /^grand piano$/i }));
    fireEvent.click(screen.getByRole("option", { name: "Warm Strings" }));

    await waitFor(() => {
      expect(actions.score.playback.slot.assignPlugin).toHaveBeenCalledWith(
        12,
        "vst3:warm-strings",
      );
    });

    expect(
      screen.getByRole("button", { name: /^grand piano$/i }),
    ).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Plugin assignment failed: Assignment failed",
    );
  });

  it("renders the refreshed rack state after slot preparation succeeds", () => {
    const initial = renderWithProviders(<PlaybackRackPanel />, {
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
    });

    expect(screen.queryByText("1. Grand Piano")).toBeNull();

    initial.unmount();

    const refreshed = renderWithProviders(<PlaybackRackPanel />, {
      appState: {
        score: {
          playback: {
            rack: {
              order: [12],
              by_id: {
                12: {
                  id: 12,
                  target: { plugin_id: "vst3:grand-piano" },
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
            editor_host_validation: null,
          },
        ],
      },
    });

    expect(refreshed.container).toBeInTheDocument();
    expect(screen.getByText("Slot 1")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /^grand piano$/i }),
    ).toBeInTheDocument();
  });

  it("does not keep host-blocked plugins selectable for slot preparation", () => {
    renderWithProviders(<PlaybackRackPanel />, {
      appState: {
        score: {
          playback: {
            rack: {
              order: [12],
              by_id: {
                12: {
                  id: 12,
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
            id: "vst3:unsupported",
            name: "Unsupported Plugin",
            bundle_path: "/Library/Audio/Plug-Ins/VST3/Unsupported Plugin.vst3",
            binary_path:
              "/Library/Audio/Plug-Ins/VST3/Unsupported Plugin.vst3/Contents/MacOS/Unsupported Plugin",
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
    });

    fireEvent.click(screen.getByRole("button", { name: /^--$/i }));

    expect(
      screen.queryByRole("option", { name: "Unsupported Plugin" }),
    ).toBeNull();
  });

  it("does not render route controls inside slot cards", () => {
    renderWithProviders(<PlaybackRackPanel />, {
      appState: {
        score: {
          playback: {
            rack: {
              order: [9],
              by_id: {
                9: {
                  id: 9,
                  target: { plugin_id: "vst3:grand-piano" },
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
            editor_host_validation: null,
          },
        ],
      },
    });

    expect(screen.queryByText("Route selected instrument")).toBeNull();
    expect(screen.queryByRole("button", { name: "Clear route" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Route here" })).toBeNull();
  });

  it("does not render the slot-card plugin UI button for blocked editor-host targets", () => {
    renderWithProviders(<PlaybackRackPanel />, {
      appState: {
        score: {
          playback: {
            rack: {
              order: [9],
              by_id: {
                9: {
                  id: 9,
                  target: { plugin_id: "vst3:grand-piano" },
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
            editor_host_validation: {
              status: "blocked",
              phase: "policy_gate",
              message: "unsupported",
              capability_evidence: null,
            },
          },
        ],
      },
    });

    expect(
      screen.queryByRole("button", { name: "Open plugin UI for Grand Piano" }),
    ).toBeNull();
  });
});
